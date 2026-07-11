# Docstring handling.
# Tests various blank line configurations around docstrings and ensures
# semicolons inside docstrings are preserved (not parsed and stripped as
# statement separators).
class BlanklineAfter:
	## Blankline after docstring

	var a = 10


class BlanklineBeforeAndAfter:

	## Blankline before and after docstring

	var a = 10


class BlanklineBefore:

	## Blankline before docstring
	var a = 10


class NoBlanklineAround:
	## No blankline around docstring
	var a = 10


class BlanklineBeforeFuncDocstring:

	## Blankline before function docstring
	##
	## The description of a variable
	func foo():
		pass


class BlanklineBetweenDocstringAndFunc:
	## Blankline between docstring and function definition

	func foo():
		pass


class DocstringAtEndOfFile:
	var a = 10

	## Docstring at the end of class


class FancyPunctuation:
	extends Object

	## How fancy the punctuation should be;
	## this may include trailing semicolons in docstrings.
	var fancy_punctuation := true

	## Another docstring with a semicolon;
	func test():
		pass
