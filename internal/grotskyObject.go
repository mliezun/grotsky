package internal

type grotskyObject struct {
	class  *grotskyClass
	fields map[string]interface{}
}

type grotskyInstance interface {
	get(tk *token) interface{}
	set(name *token, value interface{})
}

func (o *grotskyObject) get(tk *token) interface{} {
	if val, ok := o.fields[tk.lexeme]; ok {
		return val
	}
	if method := o.class.findMethod(tk.lexeme); method != nil {
		return method.bind(o)
	}
	// TODO: handle error
	return nil
}

func (o *grotskyObject) set(name *token, value interface{}) {
	o.fields[name.lexeme] = value
}
