package fuel

type Module struct {
	mass int
}

func (module Module) Fuel() int {
	max := func(x, y int) int {
		if x > y {
			return x
		}
		return y
	}
	return max((module.mass/3)-2, 0)
}

func (module Module) CompoundFuel() int {
	currentFuel := module.Fuel()
	for extraFuel := (Module{mass: currentFuel}).Fuel(); extraFuel > 0; extraFuel = (Module{mass: extraFuel}).Fuel() {
		currentFuel += extraFuel
	}
	return currentFuel
}
