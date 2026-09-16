local M = {}

local function create_floating_window(title)
  local width = math.min(90, math.floor(vim.o.columns * 0.8))
  local height = math.min(30, math.floor(vim.o.lines * 0.8))
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  local buf = vim.api.nvim_create_buf(false, true)
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].filetype = "markdown"

  local opts = {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = "rounded",
    title = " " .. (title or "CPKB Add Snippet") .. " [Save: :w or <C-s> | Quit: <Esc>] ",
    title_pos = "center",
  }

  local win = vim.api.nvim_open_win(buf, true, opts)
  return buf, win
end

local function save_snippet_from_buffer(buf, win)
  local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
  local content = table.concat(lines, "\n")

  local parts = vim.split(content, "%-%-%-", { plain = false, trimempty = false })
  if #parts < 2 then
    vim.notify("[CPKB] Error: Missing '---' separator between metadata and code.", vim.log.levels.ERROR)
    return
  end

  local meta_part = parts[1]
  table.remove(parts, 1)
  local code_part = vim.trim(table.concat(parts, "---"))

  local title = ""
  local desc = ""
  local use_case = ""
  local tags = ""
  local language = "cpp"

  for _, line in ipairs(vim.split(meta_part, "\n")) do
    local l = vim.trim(line)
    if l:lower():match("^title:") then
      title = vim.trim(l:sub(7))
    elseif l:lower():match("^description:") then
      desc = vim.trim(l:sub(13))
    elseif l:lower():match("^use%s*case:") then
      use_case = vim.trim(l:sub(10))
    elseif l:lower():match("^tags:") then
      tags = vim.trim(l:sub(6))
    elseif l:lower():match("^language:") then
      language = vim.trim(l:sub(10))
    end
  end

  if title == "" then
    vim.notify("[CPKB] Error: Title is required.", vim.log.levels.ERROR)
    return
  end
  if code_part == "" then
    vim.notify("[CPKB] Error: Code is required.", vim.log.levels.ERROR)
    return
  end

  -- Construct a temporary json file to import safely
  local snippet_payload = {
    {
      title = title,
      description = desc,
      use_case = use_case,
      tags = tags,
      language = language,
      code = code_part,
    }
  }

  local tmp_file = vim.fn.tempname() .. ".json"
  local f = io.open(tmp_file, "w")
  if not f then
    vim.notify("[CPKB] Failed to create temporary file for import.", vim.log.levels.ERROR)
    return
  end
  f:write(vim.json.encode(snippet_payload))
  f:close()

  local handle = io.popen(string.format("cpkb import %s --format json", vim.fn.shellescape(tmp_file)))
  local result = handle and handle:read("*a") or ""
  if handle then handle:close() end
  os.remove(tmp_file)

  if vim.api.nvim_win_is_valid(win) then
    vim.api.nvim_win_close(win, true)
  end

  vim.notify("[CPKB] " .. vim.trim(result or "Snippet added successfully!"), vim.log.levels.INFO)
end

local function open_add_editor(initial_code, detected_lang, mode_label)
  local buf, win = create_floating_window("CPKB Add Snippet (" .. mode_label .. ")")

  local template = {
    "Title: ",
    "Description: ",
    "Use Case: ",
    "Tags: ",
    "Language: " .. (detected_lang or "cpp"),
    "---",
  }

  for s in (initial_code or ""):gmatch("[^\r\n]+") do
    table.insert(template, s)
  end

  vim.api.nvim_buf_set_lines(buf, 0, -1, false, template)
  vim.api.nvim_win_set_cursor(win, { 1, 7 }) -- Place cursor after "Title: "

  -- Save keymaps
  local save_action = function()
    save_snippet_from_buffer(buf, win)
  end

  vim.keymap.set("n", "<C-s>", save_action, { buffer = buf, desc = "Save Snippet" })
  vim.keymap.set("i", "<C-s>", function()
    vim.cmd("stopinsert")
    save_action()
  end, { buffer = buf, desc = "Save Snippet" })
  vim.keymap.set("n", "<leader>w", save_action, { buffer = buf, desc = "Save Snippet" })
  vim.keymap.set("n", "<Esc>", function()
    if vim.api.nvim_win_is_valid(win) then
      vim.api.nvim_win_close(win, true)
    end
  end, { buffer = buf, desc = "Close window" })
  vim.keymap.set("n", "q", function()
    if vim.api.nvim_win_is_valid(win) then
      vim.api.nvim_win_close(win, true)
    end
  end, { buffer = buf, desc = "Close window" })

  -- Intercept buffer write (:w)
  vim.api.nvim_create_autocmd("BufWriteCmd", {
    buffer = buf,
    callback = function()
      save_action()
    end,
  })
end

function M.add_from_visual()
  -- Exit visual mode to set '< and '> marks
  vim.cmd([[execute "normal! \<ESC>"]])

  local start_pos = vim.api.nvim_buf_get_mark(0, "<")
  local end_pos = vim.api.nvim_buf_get_mark(0, ">")
  local start_row = start_pos[1] - 1
  local end_row = end_pos[1]

  if start_row >= end_row or start_row < 0 then
    vim.notify("[CPKB] No visual selection detected. Please select lines in visual mode.", vim.log.levels.WARN)
    return
  end

  local lines = vim.api.nvim_buf_get_lines(0, start_row, end_row, false)
  local code = table.concat(lines, "\n")
  local ft = vim.bo.filetype
  if ft == "" then ft = "cpp" end

  open_add_editor(code, ft, "Visual Selection")
end

function M.add_from_yank()
  local yanked = vim.fn.getreg('"')
  if not yanked or vim.trim(yanked) == "" then
    yanked = vim.fn.getreg("+")
  end

  if not yanked or vim.trim(yanked) == "" then
    vim.notify("[CPKB] No yanked code found. Please yank code into register first or use visual mode (<leader>cv).", vim.log.levels.WARN)
    return
  end

  local ft = vim.bo.filetype
  if ft == "" then ft = "cpp" end

  open_add_editor(yanked, ft, "Yank Register")
end

return M
