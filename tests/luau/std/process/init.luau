local process = require("@std/process")

local child = process.spawn {
    program = "seal",
    args = { "--help" }
}