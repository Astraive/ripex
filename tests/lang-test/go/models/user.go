package models

// ripex-lang-test: Go User model — struct, receiver methods, constructor.
type User struct {
	Name  string
	Email string
	Roles []string
}

func NewUser(name, email string) *User {
	return &User{Name: name, Email: email, Roles: []string{}}
}

// Receiver method — receiver type is a known silent drop in ripex.
func (u *User) Describe() string {
	return u.Name + " <" + u.Email + ">"
}

func (u User) IsAdmin() bool {
	for _, r := range u.Roles {
		if r == "admin" {
			return true
		}
	}
	return false
}
