# A leading comment stays before the unmatched end marker.
#endregion Unmatched
#region Empty
#endregion
#region Notes
# No declaration is needed to retain this region.
## This is still a comment, not a class docstring.
#endregion

signal outside_changed

var outside_value = 1

#region Unclosed
# This trailing comment stays inside the unclosed region.
