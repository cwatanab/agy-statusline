// ─── Parsed Input ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParsedInput {
    pub agent_state: String,
    pub used_percentage: f64,
    pub sandbox_enabled: bool,
    pub sandbox_allow_network: bool,
    pub artifact_count: u32,
    pub subagent_count: u32,
    pub task_count: u32,
    pub model_id: String,
    pub model_display_name: String,
    pub terminal_width: u32,
    pub working_dir: String,
    pub conversation_id: String,
    pub version: String,
    pub plan_tier: String,
    pub email: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub context_window_size: u64,
    pub turn_input_tokens: u64,
    pub turn_output_tokens: u64,
    pub gemini_5h_pct: f64,
    pub gemini_weekly_pct: f64,
    pub third_party_5h_pct: f64,
    pub third_party_weekly_pct: f64,
    pub gemini_5h_reset: i64,
    pub gemini_weekly_reset: i64,
    pub third_party_5h_reset: i64,
    pub third_party_weekly_reset: i64,
}

impl Default for ParsedInput {
    fn default() -> Self {
        ParsedInput {
            agent_state: "idle".into(),
            used_percentage: 0.0,
            sandbox_enabled: false,
            sandbox_allow_network: false,
            artifact_count: 0,
            subagent_count: 0,
            task_count: 0,
            model_id: String::new(),
            model_display_name: String::new(),
            terminal_width: 80,
            working_dir: String::new(),
            conversation_id: String::new(),
            version: String::new(),
            plan_tier: String::new(),
            email: String::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            context_window_size: 0,
            turn_input_tokens: 0,
            turn_output_tokens: 0,
            gemini_5h_pct: -1.0,
            gemini_weekly_pct: -1.0,
            third_party_5h_pct: -1.0,
            third_party_weekly_pct: -1.0,
            gemini_5h_reset: -1,
            gemini_weekly_reset: -1,
            third_party_5h_reset: -1,
            third_party_weekly_reset: -1,
        }
    }
}

// ─── JSON Parser ──────────────────────────────────────────────────────────

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        JsonParser { bytes: input.as_bytes(), pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn is_null(&mut self) -> bool {
        self.skip_whitespace();
        if self.bytes.get(self.pos..self.pos + 4) == Some(b"null") {
            self.pos += 4;
            true
        } else {
            false
        }
    }

    fn read_string(&mut self) -> &'a str {
        self.skip_whitespace();
        if self.pos >= self.bytes.len() || self.bytes[self.pos] != b'"' {
            return "";
        }
        self.advance();
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'"' {
            if self.bytes[self.pos] == b'\\' {
                self.pos += 1;
            }
            self.pos += 1;
        }
        let end = self.pos;
        if self.pos < self.bytes.len() {
            self.advance();
        }
        std::str::from_utf8(&self.bytes.get(start..end).unwrap_or(b"")).unwrap_or("")
    }

    fn read_f64(&mut self) -> f64 {
        self.read_number_str().parse().unwrap_or(0.0)
    }

    fn read_u64(&mut self) -> u64 {
        self.read_number_str().parse().unwrap_or(0)
    }

    fn read_u32(&mut self) -> u32 {
        self.read_number_str().parse().unwrap_or(0)
    }

    fn read_i64(&mut self) -> i64 {
        self.read_number_str().parse().unwrap_or(-1)
    }

    fn read_number_str(&mut self) -> &'a str {
        self.skip_whitespace();
        let start = self.pos;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'-' | b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' => self.pos += 1,
                _ => break,
            }
        }
        std::str::from_utf8(&self.bytes[start..self.pos]).unwrap_or("0")
    }

    fn read_bool(&mut self) -> bool {
        self.skip_whitespace();
        if self.pos >= self.bytes.len() {
            return false;
        }
        if self.bytes[self.pos] == b't' {
            self.pos += 4;
            true
        } else {
            self.pos += 5;
            false
        }
    }

    fn read_array_len(&mut self) -> u32 {
        self.skip_whitespace();
        self.advance();
        let mut count = 0;
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b']') {
                self.advance();
                return count;
            }
            self.skip_value();
            count += 1;
            self.skip_whitespace();
            if self.peek() == Some(b',') {
                self.advance();
            }
        }
    }

    fn skip_value(&mut self) {
        self.skip_whitespace();
        match self.peek() {
            Some(b'"') => {
                self.advance();
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'"' {
                    if self.bytes[self.pos] == b'\\' {
                        self.pos += 1;
                    }
                    self.pos += 1;
                }
                self.advance();
            }
            Some(b'{') => { self.advance(); self.skip_object(); }
            Some(b'[') => { self.advance(); self.skip_array(); }
            Some(b't') => { self.pos += 4; }
            Some(b'f') => { self.pos += 5; }
            Some(b'n') => { self.pos += 4; }
            Some(b'-' | b'0'..=b'9') => { self.read_number_str(); }
            _ => {}
        }
    }

    fn skip_object(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b'}') {
                self.advance();
                return;
            }
            self.skip_value();
            self.skip_whitespace();
            if self.peek() == Some(b':') {
                self.advance();
                self.skip_value();
            }
            self.skip_whitespace();
            if self.peek() == Some(b',') {
                self.advance();
            }
        }
    }

    fn skip_array(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b']') {
                self.advance();
                return;
            }
            self.skip_value();
            self.skip_whitespace();
            if self.peek() == Some(b',') {
                self.advance();
            }
        }
    }
}

pub fn parse_input(json: &str) -> ParsedInput {
    let json = json.trim_start_matches('\u{FEFF}');
    let mut p = JsonParser::new(json);
    p.skip_whitespace();
    if p.peek() != Some(b'{') {
        return ParsedInput::default();
    }
    p.advance();

    let mut input = ParsedInput::default();

    loop {
        p.skip_whitespace();
        match p.peek() {
            None => break,
            Some(b'}') => break,
            Some(b'"') => {
                let key = p.read_string();
                p.skip_whitespace();
                if p.peek() == Some(b':') {
                    p.advance();
                } else {
                    continue;
                }
                parse_field(&mut p, &mut input, key);
                p.skip_whitespace();
                if p.peek() == Some(b',') {
                    p.advance();
                }
            }
            _ => {
                p.skip_value();
                p.skip_whitespace();
                if p.peek() == Some(b',') {
                    p.advance();
                }
            }
        }
    }

    input
}

fn parse_field(p: &mut JsonParser, input: &mut ParsedInput, key: &str) {
    match key {
        "agent_state" => {
            if !p.is_null() { input.agent_state = p.read_string().to_string(); }
        }
        "artifact_count" => input.artifact_count = p.read_u32(),
        "task_count" => input.task_count = p.read_u32(),
        "terminal_width" => input.terminal_width = p.read_u32(),
        "cwd" => {
            if !p.is_null() { input.working_dir = p.read_string().to_string(); }
        }
        "conversation_id" => {
            if !p.is_null() { input.conversation_id = p.read_string().to_string(); }
        }
        "version" => {
            if !p.is_null() { input.version = p.read_string().to_string(); }
        }
        "plan_tier" => {
            if !p.is_null() { input.plan_tier = p.read_string().to_string(); }
        }
        "email" => {
            if !p.is_null() { input.email = p.read_string().to_string(); }
        }
        "subagents" => {
            if p.is_null() {
                input.subagent_count = 0;
            } else {
                input.subagent_count = p.read_array_len();
            }
        }
        "context_window" => { if !p.is_null() { parse_context_window(p, input); } }
        "sandbox" => { if !p.is_null() { parse_sandbox(p, input); } }
        "model" => { if !p.is_null() { parse_model(p, input); } }
        "quota" => { if !p.is_null() { parse_quota(p, input); } }
        _ => { p.skip_value(); }
    }
}

fn parse_context_window(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance();
    loop {
        p.skip_whitespace();
        if p.peek() != Some(b'"') { p.advance(); break; }
        let key = p.read_string();
        p.skip_whitespace();
        if p.peek() == Some(b':') { p.advance(); }
        match key {
            "used_percentage" => input.used_percentage = p.read_f64(),
            "total_input_tokens" => input.total_input_tokens = p.read_u64(),
            "total_output_tokens" => input.total_output_tokens = p.read_u64(),
            "context_window_size" => input.context_window_size = p.read_u64(),
            "current_usage" => {
                p.skip_whitespace();
                p.advance();
                loop {
                    p.skip_whitespace();
                    if p.peek() == Some(b'}') { p.advance(); break; }
                    match p.read_string() {
                        "input_tokens" => { p.skip_whitespace(); p.advance(); input.turn_input_tokens = p.read_u64(); }
                        "output_tokens" => { p.skip_whitespace(); p.advance(); input.turn_output_tokens = p.read_u64(); }
                        _ => { p.skip_whitespace(); p.advance(); p.skip_value(); }
                    }
                    p.skip_whitespace();
                    if p.peek() == Some(b',') { p.advance(); }
                }
            }
            _ => { p.skip_value(); }
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}

fn parse_sandbox(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance();
    loop {
        p.skip_whitespace();
        if p.peek() != Some(b'"') { p.advance(); break; }
        match p.read_string() {
            "enabled" => { p.skip_whitespace(); p.advance(); input.sandbox_enabled = p.read_bool(); }
            "allow_network" => { p.skip_whitespace(); p.advance(); input.sandbox_allow_network = p.read_bool(); }
            _ => { p.skip_whitespace(); p.advance(); p.skip_value(); }
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}

fn parse_model(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance();
    loop {
        p.skip_whitespace();
        if p.peek() != Some(b'"') { p.advance(); break; }
        match p.read_string() {
            "id" => { p.skip_whitespace(); p.advance(); if !p.is_null() { input.model_id = p.read_string().to_string(); } }
            "display_name" => { p.skip_whitespace(); p.advance(); if !p.is_null() { input.model_display_name = p.read_string().to_string(); } }
            _ => { p.skip_whitespace(); p.advance(); p.skip_value(); }
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}

fn parse_quota(p: &mut JsonParser, input: &mut ParsedInput) {
    p.skip_whitespace();
    p.advance();
    loop {
        p.skip_whitespace();
        if p.peek() != Some(b'"') { p.advance(); break; }
        let quota_key = p.read_string().to_string();
        p.skip_whitespace();
        p.advance();
        p.skip_whitespace();
        p.advance();
        let mut fraction = -1.0f64;
        let mut reset_sec = -1i64;
        loop {
            p.skip_whitespace();
            if p.peek() != Some(b'"') { p.advance(); break; }
            let entry_key = p.read_string();
            p.skip_whitespace();
            p.advance();
            match entry_key {
                "remaining_fraction" => { if !p.is_null() { fraction = p.read_f64(); } }
                "reset_in_seconds" => { if !p.is_null() { reset_sec = p.read_i64(); } }
                _ => { p.skip_value(); }
            }
            p.skip_whitespace();
            if p.peek() == Some(b',') { p.advance(); }
        }
        let pct = if fraction >= 0.0 { (fraction * 1000.0).round() / 10.0 } else { -1.0 };
        match quota_key.as_str() {
            "gemini-5h" => { input.gemini_5h_pct = pct; input.gemini_5h_reset = reset_sec; }
            "gemini-weekly" => { input.gemini_weekly_pct = pct; input.gemini_weekly_reset = reset_sec; }
            "3p-5h" => { input.third_party_5h_pct = pct; input.third_party_5h_reset = reset_sec; }
            "3p-weekly" => { input.third_party_weekly_pct = pct; input.third_party_weekly_reset = reset_sec; }
            _ => {}
        }
        p.skip_whitespace();
        if p.peek() == Some(b',') { p.advance(); }
    }
}
