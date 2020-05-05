package internal

type grotskyObject struct {
	class  *grotskyClass
	fields map[string]interface{}
}

func (o *grotskyObject) get(tk *token) interface{} {
	if val, ok := o.fields[tk.lexeme]; ok {
		return val
	}
	if method := o.class.findMethod(tk.lexeme); method != nil {
		return method
	}
	// TODO: handle error
	return nil
}

func (o *grotskyObject) set(name *token, value interface{}) {
	o.fields[name.lexeme] = value
}
