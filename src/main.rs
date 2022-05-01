#![feature(trait_alias)]
use chumsky::prelude::*;

const TEST_DATA: &'static str = include_str!("../test");
trait VdaParser<T> = Parser<char, T, Error = Simple<char>>;

fn main() {
    let result = parser().parse(TEST_DATA);
    match result {
        Ok(parsed) => println!("{:?}", parsed),
        Err(err) => println!("{:?}", err),
    }
}

#[derive(Debug)]
struct Vda {
    kunde: String,
    lieferant: String,
    werk: String,
    abladestelle: String,
    lieferabruf_alt: u32,
    lieferabruf_neu: u32,
    sachnummer: String,
    mengeeinheit: String,
    rueckstandmenge: String,
    sofortbedarf: String,
    abrufe: Vec<Abruf>,
}

#[derive(Debug)]
struct Abruf {
    date: u32,
    amount: u32,
}

fn parser() -> impl VdaParser<Vda> {
    parse_511()
        .then(parse_512())
        .then(
            parse_513()
                // 513 - 512 - 513 chains
                .then_ignore(parse_512().then(parse_513()).repeated())
                .chain(abruf_chain()),
        )
        .then_ignore(parse_515().or_not())
        .then_ignore(parse_517().or_not())
        .then_ignore(parse_518().or_not())
        .then_ignore(parse_519())
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
        });
    todo()
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
        .then_ignore(text::whitespace().repeated().exactly(83))
}

struct Result512 {
    werk_kunde: String,
    lieferabruf_neu: u32,
    lieferabruf_alt: u32,
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
        .then_ignore(n_alphanums(25))
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
        .then_ignore(any().repeated().exactly(43))
        .ignore_then(abruf())
        .repeated()
        .exactly(5)
        .then_ignore(any().repeated().exactly(6))
}

fn parse_514() -> impl VdaParser<Vec<Abruf>> {
    header(514)
        .ignore_then(abruf())
        .repeated()
        .exactly(8)
        .then_ignore(any().repeated().exactly(2))
}

fn parse_515() -> impl VdaParser<()> {
    header(515)
        .then_ignore(any().repeated().exactly(123))
        .then(parse_517().or_not())
        .ignored()
}

fn parse_517() -> impl VdaParser<()> {
    recursive(|parser| {
        header(517)
            .then_ignore(any().repeated().exactly(123))
            .then(parser.or(parse_518()))
            .ignored()
    })
}

fn parse_518() -> impl VdaParser<()> {
    header(518)
        .then_ignore(any().repeated().exactly(123))
        .then(parse_512())
        .then(parse_513())
        .ignored()
}

fn parse_519() -> impl VdaParser<()> {
    header(519)
        .then_ignore(any().repeated().exactly(123))
        .ignored()
}

fn header(code: u32) -> impl VdaParser<()> {
    just(code.to_string())
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

fn ignore_51213() -> impl VdaParser<()> {
    parse_512().then(parse_513()).ignored()
}

fn n_alphanums(n: usize) -> impl VdaParser<String> {
    filter::<_, _, Simple<char>>(|c| char::is_alphanumeric(*c))
        .repeated()
        .exactly(n)
        .collect::<String>()
}

fn counted_number(n: usize) -> impl VdaParser<u32> {
    filter::<_, _, Simple<char>>(char::is_ascii_digit)
        .repeated()
        .exactly(n)
        .collect::<String>()
        .try_map(|s, span| s.parse().map_err(|e| Simple::custom(span, e)))
}

fn limited_number(n: usize) -> impl VdaParser<u32> {
    filter::<_, _, Simple<char>>(char::is_ascii)
        .repeated()
        .exactly(n)
        .map(|s| s.iter().skip_while(|c| **c == ' ').collect::<String>())
        .try_map(|s, span| s.parse().map_err(|e| Simple::custom(span, e)))
}
