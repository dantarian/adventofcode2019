package fuel

import (
	"testing"
)

func TestFuelForShip(t *testing.T) {
	input := [4]int{12, 14, 1969, 100756}
	ship := NewShip(input[:])
	got := ship.Fuel()
	if got != 34241 {
		t.Errorf("FuelForShip([12, 14, 1969, 100756]) = %d, expected 34241", got)
	}
}

func TestCompoundFuelForShip(t *testing.T) {
	input := [3]int{14, 1969, 100756}
	ship := NewShip(input[:])
	got := ship.CompoundFuel()
	if got != 51314 {
		t.Errorf("FuelForShip([12, 14, 1969, 100756]) = %d, expected 51314", got)
	}
}
