# Await keyword spacing
# Ensures await keeps spacing after it, and that `not await` stays together
# without extra spaces.
var normal = await
	test()

var negated  :=    not await test()
