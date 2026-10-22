enum Color
  Red
  Green
  Blue
  Unknown
end

def describe(c: read Color) -> String
  match c
  when Color.Red
    return "red"
  when Color.Green, Color.Blue
    return "cool"
  else
    return "other"
  end
end

def main()
  let c: Color = Color.Green
  puts describe(c)
end
