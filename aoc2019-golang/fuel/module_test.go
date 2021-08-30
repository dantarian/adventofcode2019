package fuel

import (
	"testing"
)

func TestFuelForModule12(t *testing.T) {
	module := Module{mass: 12}
	got := module.Fuel()
	if got != 2 {
		t.Errorf("Fuel = %d; want 2", got)
	}
}

func TestFuelForModule14(t *testing.T) {
	module := Module{mass: 14}
	got := module.Fuel()
	if got != 2 {
		t.Errorf("Fuel = %d; want 2", got)
	}
}

func TestFuelForModule1969(t *testing.T) {
	module := Module{mass: 1969}
	got := module.Fuel()
	if got != 654 {
		t.Errorf("Fuel = %d; want 654", got)
	}
}

func TestFuelForModule100756(t *testing.T) {
	module := Module{mass: 100756}
	got := module.Fuel()
	if got != 33583 {
		t.Errorf("Fuel = %d; want 33583", got)
	}
}

func TestSmallModule(t *testing.T) {
	module := Module{mass: 1}
	got := module.Fuel()
	if got != 0 {
		t.Errorf("Fuel = %d; want 0", got)
	}
}

func TestCompoundFuel14(t *testing.T) {
	module := Module{mass: 14}
	got := module.CompoundFuel()
	if got != 2 {
		t.Errorf("CompoundFuel = %d; want 2", got)
	}
}

func TestCompoundFuel1969(t *testing.T) {
	module := Module{mass: 1969}
	got := module.CompoundFuel()
	if got != 966 {
		t.Errorf("CompoundFuel = %d; want 966", got)
	}
}

func TestCompoundFuel100756(t *testing.T) {
	module := Module{mass: 100756}
	got := module.CompoundFuel()
	if got != 50346 {
		t.Errorf("CompoundFuel = %d; want 50346", got)
	}
}
