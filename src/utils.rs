fn update_markdown_state(line: &str, stack: &mut Vec<String>) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // If in a code block, look only for closing code block
        if let Some(top) = stack.last() {
            if top.starts_with("```") {
                if i + 2 < chars.len() && chars[i] == '`' && chars[i+1] == '`' && chars[i+2] == '`' {
                    stack.pop();
                    i += 3;
                    continue;
                } else {
                    i += 1;
                    continue;
                }
            }
            
            // If in inline code, look only for closing inline code
            if top == "`" {
                if chars[i] == '`' && (i + 1 == chars.len() || chars[i+1] != '`') {
                    stack.pop();
                    i += 1;
                    continue;
                } else {
                    i += 1;
                    continue;
                }
            }
        }

        // Detect code block open
        if i + 2 < chars.len() && chars[i] == '`' && chars[i+1] == '`' && chars[i+2] == '`' {
            let mut lang = String::from("```");
            let mut j = i + 3;
            while j < chars.len() && chars[j].is_alphanumeric() {
                lang.push(chars[j]);
                j += 1;
            }
            stack.push(lang);
            i = j;
            continue;
        }
        
        // Detect 2-char tokens
        let mut matched_2 = false;
        if i + 1 < chars.len() {
            let token = match (chars[i], chars[i+1]) {
                ('*', '*') => Some("**"),
                ('_', '_') => Some("__"),
                ('~', '~') => Some("~~"),
                ('|', '|') => Some("||"),
                _ => None,
            };
            if let Some(t) = token {
                let t_str = t.to_string();
                if let Some(pos) = stack.iter().rposition(|x| x == &t_str) {
                    stack.remove(pos); // Close it
                } else {
                    stack.push(t_str); // Open it
                }
                i += 2;
                matched_2 = true;
            }
        }
        if matched_2 { continue; }
        
        // Detect 1-char inline code token
        if chars[i] == '`' {
            let t_str = "`".to_string();
            if let Some(pos) = stack.iter().rposition(|x| x == &t_str) {
                stack.remove(pos);
            } else {
                stack.push(t_str);
            }
        }
        i += 1;
    }
}

pub fn split_smart(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let max_len = 1900; // Leave room for formatting tags to be closed and reopened
    
    let mut stack: Vec<String> = Vec::new();

    for line in text.lines() {
        // If adding this line exceeds the limit
        if current_chunk.len() + line.len() + 1 > max_len && !current_chunk.is_empty() {
            // Close active formats in reverse order
            for tag in stack.iter().rev() {
                if tag.starts_with("```") {
                    current_chunk.push_str("\n```");
                } else {
                    current_chunk.push_str(tag);
                }
            }
            
            chunks.push(current_chunk.clone());
            current_chunk.clear();
            
            // Re-open active formats in forward order for the next chunk
            for tag in stack.iter() {
                current_chunk.push_str(tag);
                if tag.starts_with("```") {
                    current_chunk.push('\n');
                }
            }
        }

        update_markdown_state(line, &mut stack);

        if !current_chunk.is_empty() && !current_chunk.ends_with('\n') {
            current_chunk.push('\n');
        }
        current_chunk.push_str(line);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}
