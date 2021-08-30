package fuel

type Ship struct {
	modules []Module
}

func NewShip(moduleMasses []int) *Ship {
	ship := &Ship{modules: make([]Module, len(moduleMasses))}
	for i, v := range moduleMasses {
		ship.modules[i] = Module{mass: v}
	}
	return ship
}

func (ship *Ship) Fuel() (total int) {
	for _, module := range ship.modules {
		total += module.Fuel()
	}
	return
}

func (ship *Ship) CompoundFuel() (total int) {
	for _, module := range ship.modules {
		total += module.CompoundFuel()
	}
	return
}
