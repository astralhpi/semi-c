pub struct MetaData {
    pub code: String,
    pub line_break_pos: Vec<usize>
}

impl MetaData {
    pub fn new(code: String) -> MetaData {
        let line_break_pos = get_line_break_positions(&code);
        MetaData {
            code,
            line_break_pos
        }

    }
    pub fn line_column(&self, pos: usize) -> (usize, usize) {
        let mut line = 0;
        for i in self.line_break_pos.iter() {
            if pos < i + 1 {
                break;
            }
            line += 1;
        }

        if line == 0 {
            (0, pos)

        }
        else {
            (line, pos - self.line_break_pos[line - 1] - 1)
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_break_pos.len() + 1
    }
}

fn get_line_break_positions(code: &str) -> Vec<usize> {
    let mut line_breaks_pos: Vec<usize> = vec![];
    let mut i = 0;
    for c in code.chars() {
        if c == '\n' {
            line_breaks_pos.push(i);
        }
        i += 1;
    }
    line_breaks_pos 
}

#[test]
fn meta_data() {
    let code = r#"int avg(int count, int *value) {
    int i, total = 0;
    int sum = 0;
    for (i = 1; i < count; i++) {
        total = total + value[i];
    }

    return (total / count);
}

int main(void) {
    int studentNumber, count, i, sum;
    int mark[4];
    float average;
    
    count = 4;
    sum = 0;

    for (i=0; i < count; i++) {
        mark[i] = i * 30;
        sum = sum + mark[i];
        average = avg(i + 1, mark);
        if (average > 40) {
            printf("%f\n", average);
        }
    }
}
"#;
    MetaData::new(code.to_string());

}

#[test]
fn test_line_column() {
    let code = "a\nbc\ndef";
    let meta = MetaData::new(code.to_string());
    assert_eq!(vec![1, 4], meta.line_break_pos);
    assert_eq!((0,0), meta.line_column(0));
    assert_eq!((0,1), meta.line_column(1));
    assert_eq!((1,0), meta.line_column(2));
    assert_eq!((1,1), meta.line_column(3));
    assert_eq!((1,2), meta.line_column(4));
    assert_eq!((2,0), meta.line_column(5));
    assert_eq!((2,1), meta.line_column(6));
}
