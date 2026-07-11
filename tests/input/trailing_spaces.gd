# Trailing whitespace handling
# Trailing whitespace should be stripped from statements and preserved inside
# strings. Also we want to strip it after the closing triple-quote mark/after a
# closing quote.
func test_trailing_spaces() -> String:           
	var new_string: String = """  
	This is a multi-line string with trailing spaces.  
	"""
	
	return new_string    
