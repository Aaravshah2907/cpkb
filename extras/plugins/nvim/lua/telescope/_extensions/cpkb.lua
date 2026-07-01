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
        local handle = io.popen(string.format("cpkb show %s", entry.value))
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
          print("No snippet selected")
          return
        end
        
        -- Insert snippet code at cursor
        local handle = io.popen(string.format("cpkb show %s", selection.value))
        if handle then
          local result = handle:read("*a")
          handle:close()
          
          -- Extract only the code block (everything after --- Code ---)
          local code_part = result:match("%-%-%-%s*Code%s*%-%-%-%s*(.*)%s*%-%-%-%-%-%-%-%-%-%-%-%-")
          if not code_part then
             code_part = result
          end
          
          local lines = {}
          for s in code_part:gmatch("[^\r\n]+") do
            table.insert(lines, s)
          end
          
          local r, _ = unpack(vim.api.nvim_win_get_cursor(0))
          vim.api.nvim_buf_set_lines(0, r, r, false, lines)
          print("Snippet " .. selection.value .. " inserted.")
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
