extends VehicleBody

class_name Car

export(NodePath) var wheel_br
export(NodePath) var wheel_bl
export(NodePath) var wheel_fr
export(NodePath) var wheel_fl

var front_axle
var rear_axle

export(Resource) var front_tuning
export(Resource) var rear_tuning
export(Resource) var power_tuning

var _steer_gain = 5

var steer_dir = 0
var throttle = 0
var brakes = 0

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
	
func get_speed():
	var p = 0.0
	for wheel in rear_axle:
		p += pow(wheel.get_rpm(), 2)
	p = sqrt(p)
	return ((p*2*PI*rear_axle[0].wheel_radius)/60)*3.6 #P is 2pi*radius meters per minute

func _physics_process(delta):
	if Input.is_action_pressed("accelerate"):
		throttle += power_tuning.throttle_gain(get_speed()) * delta
	else:
		throttle -= power_tuning.throttle_release * delta
	
	if Input.is_action_pressed("brake"):
		brakes += power_tuning.brake_gain * delta
	else:
		brakes -= power_tuning.brake_release * delta
	
	if Input.is_action_pressed("steer_right"):
		steer_dir -= _steer_gain * delta
	elif Input.is_action_pressed("steer_left"):
		steer_dir += _steer_gain * delta
	else:
		if steer_dir < -0.1:
			steer_dir += _steer_gain * delta
		elif steer_dir > 0.1:
			steer_dir -= _steer_gain * delta
		else:
			steer_dir = 0
	
	throttle = clamp(throttle, 0, 1)
	brakes = clamp(brakes, 0, 1)
	steer_dir = clamp(steer_dir, -1, 1)
	
	engine_force = power_tuning.power(get_speed(), throttle)
	brake = power_tuning.brake_power * brakes
	
	if Input.is_action_pressed("brake"):
		if(wheel_br.get_rpm() > 3):
			brake = power_tuning.brake_power * brakes
		else:
			brake = 0
			engine_force = -power_tuning.reverse_power
		
	steering = 0.4 * steer_dir
