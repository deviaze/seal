local str = require("@std/str")

local daleki_obriyi_lyrics = [[
	"Далекі Обрії" by Ignea
	
	Неначе у золотій клітці
	Охоплена гамою барв
	Не cтерплять ми бідні зіниці
	Щодня - небуденний чар
	Міцні татуйовані лиця
	Боронять минуле своє
	Прощальне слово "верніться"
	Спіткає усюди мене
	Там де жодних листів не буває
	Скільки років кому не питають
	Полюю на кадри світу, що зникає
	Далекі обрії кличуть і манять
	Тремтіла, ламалась, благала
	Звивалась фільмова тасьма
	Дно пропасті - карна виправа
	Розсудить пралісна імла
	Коралеві скелі і мури
	Квітковий вівтар на ножах
	Тепер мене звати Теура
	Тепер я - багряний птах!
	Мої світлини не в змозі передати
	Неземної краси і омани
	Люди там не живуть - виживають
	Далекі обрії ваблять, жахають
]]

local function basichelperfunctions()
	assert(
		str.startswith(
			str.slice(
				str.trimfront(daleki_obriyi_lyrics), 0, 12
			),
			"\"Дал"
		) == true, "str oops"
	)
end

basichelperfunctions()

local function strsplit()
	local a1 = "Далекі"
	assert(str.split(a1)[3] == "л", "Далекі third index not л")
	
	local chars = str.split("seals 🦭 say hi", " ", "hi")
	assert(
		chars[1] == "seals" 
		and chars[2] == "🦭"
		and chars[3] == "say"
		and chars[4] :: any == nil, -- we had an issue with empty final string from separators
		"str.split with seal emoji not worked?"
	)
end

strsplit()

local function strsplitbutoopswehaveinvalidunicode()
	local fs = require("@std/fs")
	local nanuk_png_path = fs.path.join(".", "tests", "data", "nanuk.png")
	local png_header_buffy = fs.readbytes(nanuk_png_path, 0, 124)
	local hstring = buffer.tostring(png_header_buffy)
	local _ = str.split(hstring) -- atp we want to just make sure it doesn't fail
end

strsplitbutoopswehaveinvalidunicode()

local function strgraphemes()
	local utf8_string = "हाय मैं एक सील हूँ 🦭 (hi i am a seal 🦭)"
	-- note we have some weirdness with iterating over hindi characters w/ unicode_reader's 
	-- extended mode which i assume is the default from it using unicode_segmentation
	-- this means we can get graphemes that dont render properly in terminals
	for offset, grapheme in str.graphemes(utf8_string) do
		if offset == 7 then
			assert(grapheme == "क", "7th byte not क")
		end
	end
end

strgraphemes()