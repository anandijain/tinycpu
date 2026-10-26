function collatz_steps(n)
    steps = 0
    while n != 1
        steps += 1
        if n % 2 == 0
            n = n / 2
        else 
            n = 3n+1
        end
    end
    steps
end

collatz_steps(6)