# This test ensures that trailing white space is preserved in a multi-line
# string after the opening mark and content inside the string, but gets removed
# after the closing multiline string mark and other statements.
func test_trailing_spaces() -> String:  
	var new_string: String = """  
	This is a multi-line string with trailing spaces.  
	"""  
	
	return new_string	
