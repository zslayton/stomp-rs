pub struct StringPool {
  name: String,
  strings: Vec<String>
}

impl StringPool { 
  pub fn with_size(name: &str, number_of_strings: u32) -> StringPool {
    let strings : Vec<String> = 
      (0..number_of_strings)
      .map(|_| String::new())
      .collect();
    debug!("Creating StringPool '{}' with {} strings.", name, number_of_strings);
    StringPool {
      name: name.to_string(),
      strings: strings
    }
  }

  pub fn get(&mut self) -> String {
    let string = match self.strings.pop() {
      Some(s) => {
        debug!("Pulling from '{}', size now: {}", self.name, self.size());
        s
      },
      None => {
        debug!("'{}' empty, creating a new string.", self.name);
        String::new()
      }
    };
    string
  }

  pub fn string_from(&mut self, s: &str) -> String {
    let mut string = self.get();
    string.push_str(s);
    debug!("String acquired from '{}' is '{}'", self.name, s);
    string
  }

  pub fn put(&mut self, mut s: String) {
    debug!("Returning string '{}' to pool '{}'", s, self.name);
    s.clear();
    self.strings.push(s);
    debug!("'{}' size now {}", self.name, self.size());
  }

  pub fn size(&self) -> usize {
    self.strings.len()
  }
}
