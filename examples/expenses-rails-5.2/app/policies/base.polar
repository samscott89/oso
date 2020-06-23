manages(manager_id, employee: User) :=
    manager_id = employee.manager_id | manages(manager_id, employee.manager);
