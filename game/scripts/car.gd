extends VehicleBody

export(NodePath) var wheel_br
export(NodePath) var wheel_bl
export(NodePath) var wheel_fr
export(NodePath) var wheel_fl

var front_axle
var rear_axle

export(Resource) var front_tuning
export(Resource) var rear_tuning
export(Resource) var power_tuning

func _ready():
	wheel_br = get_node(wheel_br)
	wheel_bl = get_node(wheel_bl)
	wheel_fr = get_node(wheel_fr)
	wheel_fl = get_node(wheel_fl)
	
	front_axle = [wheel_fr, wheel_fl]
	rear_axle = [wheel_br, wheel_bl]
	
	set_front_tuning(front_tuning)
	set_rear_tuning(rear_tuning)

# tuning is an instance of axle_tuning.gd
func _set_axle_tuning(tuning, axle):
	for wheel in axle:
		wheel.wheel_roll_influence = tuning.roll
		wheel.wheel_friction_slip = tuning.slip
		wheel.wheel_rest_length = tuning.rest
		wheel.suspension_travel = tuning.travel
		wheel.suspension_stiffness = tuning.stiffness
		wheel.suspension_max_force = tuning.max_force
		wheel.damping_compression = tuning.compression
		wheel.damping_relaxation = tuning.relaxation
			
func set_front_tuning(tuning):
	front_tuning = tuning
	_set_axle_tuning(tuning, front_axle)
	
func set_rear_tuning(tuning):
	rear_tuning = tuning
	_set_axle_tuning(tuning, rear_axle)

func _physics_process(_delta):
	if Input.is_action_pressed("accelerate"):
		engine_force = 150
	elif Input.is_action_pressed("brake"):
		if(wheel_br.get_rpm() > 5):
			brake = 20
			engine_force = 0
		else:
			brake = 0
			engine_force = -50
	else:
		engine_force = 0
		brake = 0
		
	steering = 0
	if Input.is_action_pressed("steer_right"):
		steering -= 0.4;
	if Input.is_action_pressed("steer_left"):
		steering += 0.4;
