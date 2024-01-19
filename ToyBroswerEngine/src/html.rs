use std::collections::HashMap;

pub fn parse(source: String) -> ToyBroswerEngine::dom::Node {
    let mut nodes = Parser { pos: 0, input: source}.parse_nodes();

    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        ToyBroswerEngine::dom::elem("html".to_owned(), HashMap::new(), nodes)
    }
}

pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    pub fn parse_nodes(&mut self) -> Vec<ToyBroswerEngine::dom::Node> {
        let mut nodes = vec!();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</"){
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    //parse single node
    pub fn parse_node(&mut self) -> ToyBroswerEngine::dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text()
        }
    }

    // parsing a single element, including open tag , contents and closing tag
    pub fn parse_element(&mut self) -> ToyBroswerEngine::dom::Node {
        // open tag
        assert_eq!(self.consume_char(), '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert_eq!(self.consume_char(), '>');
        // contents
        let children = self.parse_nodes();
        // closing tag
        assert_eq!(self.consume_char(), '<');
        assert_eq!(self.consume_char(), '/');
        assert_eq!(self.parse_tag_name(), tag_name);
        assert_eq!(self.consume_char(), '>');

        ToyBroswerEngine::dom::elem(tag_name, attrs, children)
    }

    pub fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false
        })
    }

    // parse a list of name = value pairs, seperated by whitespace
    pub fn parse_attributes(&mut self) -> ToyBroswerEngine::dom::AtterMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        attributes
    }
    // parse a single name = value pair
    pub fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert_eq!(self.consume_char(), '=');
        let value = self.parse_attr_value();
        (name, value)
    }
    // parse a quoted value
    pub fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert_eq!(self.consume_char(), open_quote);
        value
    }
    // parses a text node
    pub fn parse_text(&mut self) -> ToyBroswerEngine::dom::Node {
        ToyBroswerEngine::dom::text(self.consume_while(|c| c != '<'))
    }   

    pub fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }
    // consumes characters until the test returns false.
    pub fn consume_while<F>(&mut self, test: F) -> String
        where F: Fn(char) -> bool {
            let mut result = String::new();
            while !self.eof() && test(self.next_char()){
                result.push(self.consume_char());
            }
            result
    }
    // return the current character and advance self.pos to the next charaxter
    pub fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos ..].char_indices();
        let (_, cur_char) =  iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    // read the current character without consuming it
    pub fn next_char(&self) -> char {
        self.input[self.pos ..].chars().next().unwrap()
    }
    // does the current input start with the given string
    pub fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos ..].starts_with(s)
    }
    // returns true if the input is consumed.
    pub fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
  
}