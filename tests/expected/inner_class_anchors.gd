#ANCHOR:class_state_machine
class StateMachine extends Node:
	#END:class_state_machine

	## A Dictionary of dictionaries. Maps State → Event → State. [br]
	## Uses [AI.State] keys. Each value is a dictionary with
	## [constant AI.Event] keys and [AI.State] values.
	#ANCHOR:fsm_var_transitions
	var transitions := { }:
		set = set_transitions
	#END:fsm_var_transitions

	## Holds the current state of the state machine.
	#ANCHOR:fsm_var_current_state
	var current_state: State
	#END:fsm_var_current_state

	## If [code]true[/code], associated mob will display a label with its current
	## state.
	#ANCHOR:fsm_var_is_debugging
	var is_debugging := false:
		set = set_is_debugging
