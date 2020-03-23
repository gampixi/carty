extends Resource

class_name PowerTuning

export var top_speed = 100.0
export var acceleration = 250.0
export var base_throttle_gain = 0.2
export var speed_throttle_gain_mult = 0.1
export var throttle_release = 0.8
export var power_gain_curve = 2.0 #-x^a + 1 [0;1]
export var brake_gain = 0.7
export var brake_release = 1.6
export var brake_power = 15.0
export var reverse_power = 50.0

func power(speed, throttle):
	var k = speed / top_speed
	k = clamp(k, 0, 1)
	
	return (1 - (pow(k, power_gain_curve))) * acceleration * throttle
	
func throttle_gain(speed):
	return base_throttle_gain + (speed * speed_throttle_gain_mult)
