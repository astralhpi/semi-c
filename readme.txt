# Introduction
 semi-c는 c언어의 서브셋 언어를 실행할 수 있는 rust로 작성된 인터프리터입니다.
카이스트 2017 가을학기 CS420 텀 프로젝트로 작성 되었으며, 아래 코드를 실행할 수 있습니다.

'''c
int avg(int count, int *value) {
    int i, total;
    total = 0;
    for (i = 1; i < count; i++) {
        int a;
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
    met = 1;

    for (i=0; i < count; i++) {
        mark[i] = i * 30;
        sum = sum + mark[i];
        average = avg(i + 1, mark);
        if (average > 40) {
            printf("%f\n", average);
        }
    }
}
'''

# 환경 설정
이 프로젝트를 빌드하려면 Rust와 Rust의 빌드 툴인 Cargo가 필요합니다.

https://www.rust-lang.org/ko-KR/install.html

해당 url의 설명을 따르면 Rust와 Cargo를 설치할 수 있습니다.

# 빌드와 테스트

주의 사항: 이 프로젝트에서 사용한 parser generator인 lalrpop 때문에 컴파일 속도가 매우 느립니다.
수 분 이상 걸릴 수 있으니, 양해 바랍니다.

## 테스트
 이 프로젝트의 유닛 테스트를 실행하려면 프로젝트 폴더에서 아래 명령어를 실행해줍니다.

 '''bash
 > cargo test
 '''

## 실행
  이 프로젝트를 빌드하고 실행하려면 프로젝트 폴더에서 아래 명령어를 실행해줍니다.

'''bash
> cargo run <소스코드 경로>
'''

# 추가 기능
## 
