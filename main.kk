import std/os/file
import std/os/path

import vdaparser


fun main()
  //print(read-text-file(path("test")) ++ "\n")
  val parsed = "test".path().read-text-file().slice().parse(pvda)

  match parsed
    ParseOk(v, _) -> v.println()
    ParseError(err, _) ->
      println("ParseError: ")
      err.println()