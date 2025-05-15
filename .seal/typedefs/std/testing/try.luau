export type try = (protected_f: () -> any?) -> {
	ok: boolean,
	match: (self: any, matcher: {
		ok: (result: any?) -> any?,
		err: (err: any?) -> any?,
	}) -> any?,
	--- expects no error to occur and returns the result of protected_function; why would you use this in a try?
	unwrap: (self: any, default: any?) -> any,
	--- expects a specific error to occur and verifies that it does
	expect_err: (self: any, handler: string | (err: unknown) -> any) -> any,
}

return (function()
	return nil :: any
end) :: try
