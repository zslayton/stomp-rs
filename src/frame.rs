use headers::HeaderList;

pub struct Frame {
  pub command : ~str,
  pub headers : HeaderList, 
  pub body : ~[u8]
}
