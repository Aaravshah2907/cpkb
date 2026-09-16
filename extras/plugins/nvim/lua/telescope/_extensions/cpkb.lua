local has_telescope, telescope = pcall(require, "telescope")
if not has_telescope then
  error("This plugin requires nvim-telescope/telescope.nvim")
end

local pickers = require("telescope.pickers")
local finders = require("telescope.finders")
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require("telescope.actions.state")
local previewers = require("telescope.previewers")

local function normalize_lang(lang)
  if not lang or lang == "" then return "text" end
  lang = lang:lower():gsub("%s+", "")
  if lang == "c++" or lang == "cc" or lang == "cxx" then return "cpp" end
  if lang == "py" then return "python" end
  if lang == "js" then return "javascript" end
  if lang == "ts" then return "typescript" end
  if lang == "rs" then return "rust" end
  if lang == "sh" or lang == "zsh" or lang == "bash" then return "sh" end
  if lang == "markdown" or lang == "md" then return "markdown" end
  if lang == "txt" or lang == "plaintext" then return "text" end
  if lang == "latex" or lang == "plaintex" or lang == "tex" then return "tex" end
  return lang
end

local function is_text_based(lang)
  local n = normalize_lang(lang)
  return n == "text" or n == "markdown" or n == "md" or n == "txt" or n == "plaintext" or n == "any"
end

local function insert_code_lines(code)
  local lines = {}
  for s in code:gmatch("[^\r\n]+") do
    table.insert(lines, s)
  end
  if #lines == 0 then return end
  local r, _ = unpack(vim.api.nvim_win_get_cursor(0))
  vim.api.nvim_buf_set_lines(0, r, r, false, lines)
end

local cpkb_search = function(opts)
  opts = opts or {}
  
  -- Fetch snippets using the scripting-friendly query command
  local fetch_snippets = function()
    local handle = io.popen("cpkb query '' --limit 1000")
    if not handle then return {} end
    local result = handle:read("*a")
    handle:close()
    
    local snippets = {}
    for line in result:gmatch("[^\r\n]+") do
      local id, title = line:match("([^|]+)%s*|%s*(.+)")
      if id and title then
        id = id:gsub("^%s*(.-)%s*$", "%1")
        title = title:gsub("^%s*(.-)%s*$", "%1")
        table.insert(snippets, {
          id = id,
          title = title,
          display = string.format("%-10s %s", id, title),
          ordinal = string.format("%s %s", id, title),
        })
      end
    end
    return snippets
  end

  local snippets = fetch_snippets()

  pickers.new(opts, {
    prompt_title = "CPKB Snippets",
    finder = finders.new_table({
      results = snippets,
      entry_maker = function(entry)
        return {
          value = entry.id,
          display = entry.display,
          ordinal = entry.ordinal,
        }
      end,
    }),
    sorter = conf.generic_sorter(opts),
    previewer = previewers.new_buffer_previewer({
      title = "Snippet Preview",
      define_preview = function(self, entry, status)
        local handle = io.popen(string.format("cpkb show %s", vim.fn.shellescape(entry.value)))
        if handle then
          local result = handle:read("*a")
          handle:close()
          local lines = {}
          for s in result:gmatch("[^\r\n]+") do
            table.insert(lines, s)
          end
          vim.api.nvim_buf_set_lines(self.state.bufnr, 0, -1, false, lines)
          vim.bo[self.state.bufnr].filetype = 'markdown'
        end
      end
    }),
    attach_mappings = function(prompt_bufnr, map)
      actions.select_default:replace(function()
        actions.close(prompt_bufnr)
        local selection = action_state.get_selected_entry()
        if not selection then
          vim.notify("[CPKB] No snippet selected", vim.log.levels.WARN)
          return
        end
        
        -- Fetch structured snippet data
        local handle = io.popen(string.format("cpkb show %s --json", vim.fn.shellescape(selection.value)))
        local json_str = handle and handle:read("*a") or ""
        if handle then handle:close() end

        local ok, data = pcall(vim.json.decode, json_str)
        if not ok or not data or not data.code then
          -- Fallback if JSON decoding fails
          local fallback_handle = io.popen(string.format("cpkb show %s", vim.fn.shellescape(selection.value)))
          local fallback_result = fallback_handle and fallback_handle:read("*a") or ""
          if fallback_handle then fallback_handle:close() end
          local code_part = fallback_result:match("%-%-%-%s*Code%s*%-%-%-%s*(.*)%s*%-%-%-%-%-%-%-%-%-%-%-%-") or fallback_result
          insert_code_lines(code_part)
          vim.notify("[CPKB] Snippet " .. selection.value .. " inserted.", vim.log.levels.INFO)
          return
        end

        local current_ft = vim.bo.filetype
        local snippet_lang = data.language or "cpp"

        -- Language validation check
        if is_text_based(snippet_lang) or normalize_lang(snippet_lang) == normalize_lang(current_ft) then
          insert_code_lines(data.code)
          vim.notify(string.format("[CPKB] Snippet %s (%s) inserted.", selection.value, snippet_lang), vim.log.levels.INFO)
        else
          -- Language mismatch prompt
          vim.notify(string.format("[CPKB] Language mismatch: snippet is '%s' but current buffer is '%s'", snippet_lang, current_ft), vim.log.levels.WARN)
          vim.ui.select({ "Cancel", "Insert Anyway" }, {
            prompt = string.format("Snippet is [%s] but buffer is [%s]. Insert anyway?", snippet_lang, current_ft),
          }, function(choice)
            if choice == "Insert Anyway" then
              insert_code_lines(data.code)
              vim.notify(string.format("[CPKB] Snippet %s inserted despite mismatch.", selection.value), vim.log.levels.INFO)
            end
          end)
        end
      end)
      return true
    end,
  }):find()
end

return telescope.register_extension({
  exports = {
    cpkb = cpkb_search
  }
})
