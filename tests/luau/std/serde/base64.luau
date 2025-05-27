local fs = require("@std/fs")
local base64 = require("@std/serde/base64")

local nanuk_buffy = fs.readbytes("./tests/data/nanuk.png")
local sensible_nanuk = base64.encode(nanuk_buffy)

local function standardserdeworks()
	local nanuk_encoded_64 = base64.encode(fs.readbytes("./tests/data/nanuk.png"))
	local nanuk_decoded_64 = base64.decode(nanuk_encoded_64)
	assert(buffer.tostring(nanuk_buffy) == buffer.tostring(nanuk_decoded_64), "nanuks don't match")
end

standardserdeworks()

local function urlsafe()
	local urlsafe_nanuk = base64.urlsafe.encode(nanuk_buffy)
	assert(#urlsafe_nanuk == 2395220, "#urlsafe_nanuk not 2395220? did he get bigger again?")
	local decoded_urlsafe_nanuk = base64.urlsafe.decode(urlsafe_nanuk)	
	assert(buffer.tostring(nanuk_buffy) == buffer.tostring(decoded_urlsafe_nanuk), "decoded urlsafe nanuk doesn't match original buffer contents")
end

urlsafe()