export type Aes = {
	generatekey: ()	-> string,
	encrypt: (plaintext: string, key: string) -> string,
	decrypt: (ciphertext: string, key: string) -> string,
}

return {} :: Aes