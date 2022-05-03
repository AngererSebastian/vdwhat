#![feature(trait_alias)]
use chumsky::prelude::*;

const TEST_DATA: &'static str = include_str!("../test");
trait VdaParser<T> = Parser<char, T, Error = Simple<char>>;

fn main() {
    //let test = "123456789";
    //let result = parser().parse(TEST_DATA);
    //let result = ignore_n(9).then(end()).parse(test);
    /*match result {
        Ok(parsed) => println!("parsed {:?}", parsed),
        Err(err) => err.iter().for_each(|e| {
            println!("{:?}", e);
        }),
    };*/
    println!("{:#?}", parse(TEST_DATA))
}

#[derive(Debug)]
#[allow(dead_code)]
struct Vda<'a> {
    kunde: &'a str,
    lieferant: &'a str,
    werk: &'a str,
    abladestelle: &'a str,
    lieferabruf_alt: u64,
    lieferabruf_neu: u64,
    sachnummer: &'a str,
    mengeeinheit: &'a str,
    rueckstandmenge: &'a str,
    sofortbedarf: &'a str,
    abrufe: Vec<Abruf>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Abruf {
    date: String,
    amount: String,
}

fn parse(input: &'_ str) -> Vda<'_> {
    let mut lines = input.lines();
    let begin: &str = lines.next().unwrap();
    
    assert_eq!(&begin[0..3], "511");

    let kunde= &begin[5..14];
    let lieferant = &begin[14..23];

    let snd = lines.next().unwrap();
    
    assert_eq!(&snd[0..3], "512");

    let werk= &snd[5..8];
    let lieferabruf_neu = to_number(&snd[8..17]);
    let lieferabruf_alt = to_number(&snd[23..32]);
    let sachnummer = &snd[38..60];
    let abladestelle = &snd[94..99];
    let mengeeinheit = &snd[103..105];

    let abrufe = parse_abrufe(lines);

    Vda {
        kunde,
        lieferant,
        werk,
        lieferabruf_neu,
        lieferabruf_alt,
        sachnummer,
        abladestelle,
        mengeeinheit,
        rueckstandmenge: "todo",
        sofortbedarf: "todo",
        abrufe
    }
}

fn parse_abrufe<'a, I: Iterator<Item = &'a str>>(inp: I) -> Vec<Abruf> {
    inp.filter(|l| &l[0..3] == "513" || &l[0..3] == "514")
        .map(|l| parse_513().or(parse_514()).parse(l))
        .map(|r| r.unwrap())
        .flatten()
        .collect::<Vec<Abruf>>()

}

fn to_number(inp: &str) -> u64 {
    inp.trim_end().parse().unwrap()
}

fn parse_513() -> impl VdaParser<Vec<Abruf>> {
    header(513)
        .then_ignore(ignore_n(43))
        .ignore_then(abruf().repeated().exactly(5))
        .then_ignore(ignore_n(5)) 
}

fn parse_514() -> impl VdaParser<Vec<Abruf>> {
    header(514)
        .ignore_then(abruf().repeated().exactly(8))
        .then_ignore(ignore_n(3)) 
}

fn header(code: u64) -> impl VdaParser<()> {
    just(code.to_string())
        .labelled("header")
        .then_ignore(counted_number(2))
        .map(|_| ())
}

fn abruf() -> impl VdaParser<Abruf> {
    n_alphanums(6)
        .then(n_alphanums(9))
        .map(|(date, amount)| Abruf { date, amount })
}

fn ignore_n(n: usize) -> impl VdaParser<()> {
    any().repeated().exactly(n).ignored()
}

fn n_alphanums(n: usize) -> impl VdaParser<String> {
    filter::<_, _, Simple<char>>(|c| char::is_ascii(c))
        .labelled("n alphanums")
        .repeated()
        .exactly(n)
        .collect::<String>()
        .map(|c| String::from(c))
}

fn counted_number(n: usize) -> impl VdaParser<u64> {
    filter::<_, _, Simple<char>>(|c| char::is_ascii_digit(c) || *c == ' ')
        .labelled("counted number")
        .repeated()
        .exactly(n)
        .collect::<String>()
        .try_map(|s, span| s.trim_end().parse().map_err(|e| Simple::custom(span, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counted_number_1() {
        let input = "12 34";
        let rslt = counted_number(4).parse(input);

        rslt.unwrap_err();
    }

    #[test]
    fn counted_number_2() {
        let input = "1234";
        let rslt = counted_number(4).parse(input).unwrap();

        assert_eq!(1234, rslt)
    }

    #[test]
    fn counted_number_3() {
        let input = "1234";
        let rslt = counted_number(5).parse(input);

        rslt.unwrap_err();
    }
    
    #[test]
    fn test_514() {
        let input = "51401140605000003000140606000000000140627000000000140630000003000140722000003000140813000003000140814000000000140904000003000   \n";
        let rslt = parse_514().then(end()).parse(input);

        rslt.unwrap();
    }

    #[test]
    fn test_514_2() {
        let input = "51401140905000000000140926000003000140930000000000555555000000000141000000003000141100000006000141200000001559000000            \n";
        let rslt = parse_514().then(end()).parse(input);

        rslt.unwrap();
    }
}