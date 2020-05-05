package internal

type grotskyClass struct {
	name          string
	superclass    *grotskyClass
	methods       []*grotskyFunction
	staticMethods []*grotskyFunction
}
