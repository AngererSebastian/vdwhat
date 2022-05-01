#![feature(trait_alias)]
use ariadne::{Label, Report, ReportKind, Source};
use chumsky::prelude::*;

const TEST_DATA: &'static str = include_str!("../test");
trait VdaParser<T> = Parser<char, T, Error = Simple<char>>;

fn main() {
    //let test = "123456789";
    let result = parser().parse(TEST_DATA);
    //let result = ignore_n(9).then(end()).parse(test);
    match result {
        Ok(parsed) => println!("parsed {:?}", parsed),
        Err(err) => err.iter().for_each(|e| {
            /*Report::build::<()>(ReportKind::Error, (), e.span().start)
            .with_message::<String>(format!("{:?}", e.reason()))
            .with_label(Label::new(e.span()))
            .finish()
            .print(Source::from(TEST_DATA))
            .unwrap();*/
            println!("{:?}", e);
        }),
    };
}

#[derive(Debug)]
struct Vda {
    kunde: String,
    lieferant: String,
    werk: String,
    abladestelle: String,
    lieferabruf_alt: u64,
    lieferabruf_neu: u64,
    sachnummer: String,
    mengeeinheit: String,
    rueckstandmenge: String,
    sofortbedarf: String,
    abrufe: Vec<Abruf>,
}

#[derive(Debug)]
struct Abruf {
    date: u64,
    amount: u64,
}

fn parser() -> impl VdaParser<Vda> {
    parse_511()
        .then(parse_512())
        .then(
            parse_513()
                // 513 - 512 - 513 chains
                //.then_ignore(parse_512().then(parse_513()).repeated())
                .chain(abruf_chain()),
        )
        .then_ignore(parse_515().or_not())
        .then_ignore(parse_517().or_not())
        .then_ignore(parse_518().or_not())
        .then_ignore(parse_519())
        .then_ignore(end())
        .map(|((r511, r512), abrufe)| Vda {
            kunde: r511.kunde,
            lieferant: r511.lieferant,
            werk: r512.werk_kunde,
            abladestelle: r512.abladestelle,
            lieferabruf_alt: r512.lieferabruf_alt,
            lieferabruf_neu: r512.lieferabruf_neu,
            sachnummer: r512.sachnummer_kunde,
            mengeeinheit: r512.mengeneinheit,
            rueckstandmenge: String::from("TODO"),
            sofortbedarf: String::from("TODO"),
            abrufe,
        })
}

struct Result511 {
    kunde: String,
    lieferant: String,
}

fn parse_511() -> impl VdaParser<Result511> {
    header(511)
        .ignore_then(n_alphanums(9))
        .then(n_alphanums(9))
        .map(|(kunde, lieferant)| Result511 { kunde, lieferant })
        .then_ignore(ignore_n(106)) // + linefeed alway here
}

struct Result512 {
    werk_kunde: String,
    lieferabruf_neu: u64,
    lieferabruf_alt: u64,
    sachnummer_kunde: String,
    abladestelle: String,
    mengeneinheit: String,
}

fn parse_512() -> impl VdaParser<Result512> {
    header(512)
        .ignore_then(n_alphanums(3)) // werk kunde
        .then(counted_number(9)) // liefer abruf neu
        .then_ignore(counted_number(6)) // lieferdatum neu
        .then(counted_number(9)) // liefer abruf alt
        .then_ignore(counted_number(6)) // lieferdatum alt
        .then(n_alphanums(22)) // sachnummer kunde
        .then_ignore(n_alphanums(22)) // sachnummer lieferant
        .then_ignore(counted_number(10)) // bestellnummer
        .then(n_alphanums(5)) //abladestelle
        .then_ignore(n_alphanums(4)) // zeichen kunde
        .then(n_alphanums(2)) // mengeneinheit
        .then_ignore(n_alphanums(26)) // + linefeed
        .map(
            |(
                (
                    (((werk_kunde, lieferabruf_neu), lieferabruf_alt), sachnummer_kunde),
                    abladestelle,
                ),
                mengeneinheit,
            )| Result512 {
                werk_kunde,
                lieferabruf_neu,
                lieferabruf_alt,
                sachnummer_kunde,
                abladestelle,
                mengeneinheit,
            },
        )
}

fn parse_513() -> impl VdaParser<Vec<Abruf>> {
    header(513)
        .then_ignore(ignore_n(43))
        .ignore_then(abruf().repeated().exactly(5))
        .then_ignore(ignore_n(6)) // linefeed
}

fn parse_514() -> impl VdaParser<Vec<Abruf>> {
    header(514)
        .ignore_then(abruf().repeated().exactly(8))
        .then_ignore(ignore_n(4)) // linefeed
}

fn parse_515() -> impl VdaParser<()> {
    header(515)
        .then_ignore(ignore_n(124)) // linefeed
        .then(parse_517().or_not())
        .ignored()
}

fn parse_517() -> impl VdaParser<()> {
    recursive(|parser| {
        header(517)
            .then_ignore(ignore_n(124)) // linefeed
            .then(parser.or(parse_518()))
            .ignored()
    })
}

fn parse_518() -> impl VdaParser<()> {
    header(518)
        .then_ignore(ignore_n(124)) // linefeed
        .then(parse_512())
        .then(parse_513())
        .ignored()
}

fn parse_519() -> impl VdaParser<()> {
    header(519).then_ignore(ignore_n(124)).ignored()
}

fn header(code: u64) -> impl VdaParser<()> {
    just(code.to_string())
        .labelled("header")
        .then_ignore(counted_number(2))
        .map(|_| ())
}

fn abruf() -> impl VdaParser<Abruf> {
    counted_number(6)
        .then(counted_number(9))
        .map(|(date, amount)| Abruf { date, amount })
}

fn abruf_chain() -> impl VdaParser<Vec<Abruf>> {
    parse_514()
        .then_ignore(parse_515().or_not())
        .repeated()
        .map(|vs| vs.into_iter().flatten().collect())
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
}
