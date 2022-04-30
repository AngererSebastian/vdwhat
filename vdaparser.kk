pub import std/text/parse

pub struct vda
  kunde: int
  lieferant: int
  werk: int
  abladestelle: int
  lieferabruf-alt: int
  lieferabruf-neu: int
  sachnummer: string
  mengeeinheit: string
  rueckstandmenge: string
  sofortbedarf: string
  abrufe: list<abruf>

pub struct abruf
  date: int
  amount: int


pub fun pvda(): <parse, exn> string 
  whitespace()
  p511()
  p512()
  p513()
  p514.many()
  p515()
  p517.many()
  p518.many()
  p519()
  whitespace()
  parse-eof()

fun p511(): <parse, exn> string
  pheader("511")
  val kunde = count(9, alpha-num)
  val lieferant = count(9, alpha-num)
  count(10, digit) // uebertragungsnummer (alt + neu)
  count(12, digit)
  count(83, whitespace)
  "TODO"


fun p512(): parse string
  pheader("512")
  val werk-kunde = count(3, alpha-num).string()
  val liefer-abruf-neu = count(9, digit)
  count(6, digit) // lieferdatum neu
  val liefer-abruf-alt = count(9, digit)
  count(6, digit) // lieferdatum alt
  val sachnummer-kunde = count(22, alpha-num)
  count(22, alpha-num) // sachnummer lieferant
  count(10, digit) // bestellnumber
  val abladestelle = count(5, alpha-num)
  count(4, alpha-num) // zeichen kunde
  val mengeeinheit = count(2, alpha-num)
  alpha-num() // anlieferungs intervall
  digit() // fertigungs freigabe
  digit() // material freigabe
  alpha-num() // verwendungsschluessel
  count(7, alpha-num) // kontinierungs schluessel
  count(7, alpha-num) // lager
  count(7, alpha-num) // leer
  "512"

fun p513(): parse string
  pheader("513")
  count(6, digit) // erfassungsdatum
  count(8, digit) // lieferscheinnummer
  count(6, digit) // lieferscheindatum
  count(12, digit) // menge
  count(10, digit) // eingangs-fortschrittszahl

  count(80, alpha-num) // skip it

  "513"


fun p514(): parse string
  pheader("514")
  count(123, alpha-num)
  "514"

fun p515(): parse string
  pheader("515")
  "515"

fun p516(): parse string
  pheader("516")
  "516"

fun p517(): parse string
  pheader("517")
  "517"

fun p518(): parse string
  pheader("518")
  "518"

fun p519(): parse string
  pheader("519")
  "519"

fun pheader(message: string): parse ()
  pstring(message)
  count(2, digit)

fun pcounted-number(digitsCount: int): <parse, exn> int
  count(digitsCount, digit)
    .foldl1() fn(acc, n)
      acc * 10 + n

fun plimit-number(limit: int): parse int
  val parsed = count(limit, alpha-num)
    .drop-while() fn (c)
      c == ' '
    .string()
    .parse-int()

  match parsed
    Just(n) -> n
    Nothing -> fail("invalid number")