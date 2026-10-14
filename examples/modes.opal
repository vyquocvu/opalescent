def inspect(msg: read String)
  puts msg
end

def consume(msg: take String)
  puts "consumed"
end

def main()
  let msg: String = "Hello Opalescent"
  inspect(msg)
  consume(msg)
end
