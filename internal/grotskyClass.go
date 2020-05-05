package internal

type grotskyClass struct {
	name          string
	superclass    *grotskyClass
	methods       map[string]*grotskyFunction
	staticMethods map[string]*grotskyFunction
}

func (c *grotskyClass) findMethod(name string) *grotskyFunction {
	if method, ok := c.methods[name]; ok {
		return method
	}
	if c.superclass != nil {
		return c.superclass.findMethod(name)
	}

	// TODO: handle error
	return nil
}
