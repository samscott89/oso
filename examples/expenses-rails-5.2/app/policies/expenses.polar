# TODO(gj): clean this up with groups
allow(_: User{id: viewer_id}, "index", __: Expense{user_id: claimant_id}) :=
    viewer_id == claimant_id | manages(viewer_id, new User{id: claimant_id});

allow(_: User{id: viewer_id}, "show", __: Expense{user_id: claimant_id}) :=
    viewer_id == claimant_id | manages(viewer_id, new User{id: claimant_id});

