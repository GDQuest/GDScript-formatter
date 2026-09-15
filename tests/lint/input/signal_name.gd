#region Good

signal public_signal(var1: int, var2: String)
signal _pseudo_private(var1: int, var2: String)
signal __also_valid_pseudo_private(var1: int, var2: String)

#endregion

#region Bad

signal BadSignal(var1: int, var2: String)
signal badSignal(var1: int, var2: String)

#endregion
