local function try<T>(f: (() -> T))
    local success, result = pcall(f)
    if success then
        print(`success: got {result}`)
    end
    local function get(s: boolean, v: T)
        return {
            s = s,
            v = v
        }
    end
    return {
        expect = function<T>(self: any, val: T, fail: string)
            -- if result == val then
            --     return true
            -- else
            --     warn(`assertion failed: {fail}`)
            --     return false
            -- end
            if result ~= val then
                warn(`assertion failed: {fail}`)
            end
            return get(success == val, val)
        end,
        throws = function(self: any, contains: string)
            if typeof(result) ~= "error" then
                warn(`assertion failed: result not an error; expected error that contains {contains}`)
                return get(false, result)
            end
            if string.find(tostring(result), contains, 1, true) then
                return get(true, result)
            else
                warn(`assertion failed, error does not contain {contains}; got: {tostring(result)}`)
                -- return false
                return get(false, result)
            end
        end
    }
end

return try