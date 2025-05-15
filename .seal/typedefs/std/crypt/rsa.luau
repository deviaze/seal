type RsaKeys = {
	public: string,
	private: string
}

export type Rsa = {
	generatekeys: () -> RsaKeys,
	encrypt: (plaintext: string, public_key: string) -> string,
	decrypt: (ciphertext: string, private_key: string) -> string,
}

return {} :: Rsa