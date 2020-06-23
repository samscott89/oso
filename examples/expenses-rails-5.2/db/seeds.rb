# frozen_string_literal: true

require 'faker'

# This file should contain all the record creation needed to seed the database with its default values.
# The data can then be loaded with the rails db:seed command (or created alongside the database with db:setup).

nyc = Location.create({ name: 'NYC' })
milwaukee = Location.create({ name: 'Milwaukee' })
crestone = Location.create({ name: 'Crestone' })

foo_industries = Organization.create({ name: 'Foo Industries' })

foo_eng = Team.create({ organization: foo_industries, name: 'Engineering' })
foo_leadership = Team.create({ organization: foo_industries, name: 'Leadership' })
foo_finance = Team.create({ organization: foo_industries, name: 'Finance' })
foo_sales = Team.create({ organization: foo_industries, name: 'Sales' })

admin = Role.create({ title: 'admin' })
accountant = Role.create({ title: 'accountant' })
employee = Role.create({ title: 'employee' })

alice = User.new({ email: 'alice@foo.com', title: 'CEO' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_leadership
  u.location = nyc
  u.roles << admin
  u.save
end
keri = User.new({ email: 'keri@foo.com', title: 'CFO' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_leadership
  u.location = crestone
  u.roles << admin
  u.manager = alice
  u.save
end
bhavik = User.new({ email: 'bhavik@foo.com', title: 'Senior Accountant' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_finance
  u.location = nyc
  u.roles << accountant
  u.manager = keri
  u.save
end
cora = User.new({ email: 'cora@foo.com', title: 'Accountant' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_finance
  u.location = nyc
  u.roles << accountant
  u.manager = bhavik
  u.save
end
deirdre = User.new({ email: 'deirdre@foo.com', title: 'Director of Engineering' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_eng
  u.location = nyc
  u.roles << employee
  u.manager = alice
  u.save
end
ebrahim = User.new({ email: 'ebrahim@foo.com', title: 'Engineering Manager' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_eng
  u.location = nyc
  u.roles << employee
  u.manager = deirdre
  u.save
end
frantz = User.new({ email: 'frantz@foo.com', title: 'Software Engineer' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_eng
  u.location = nyc
  u.roles << employee
  u.manager = ebrahim
  u.save
end
greta = User.new({ email: 'greta@foo.com', title: 'Director of Sales' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_sales
  u.location = nyc
  u.roles << employee
  u.manager = alice
  u.save
end
han = User.new({ email: 'han@foo.com', title: 'Regional Sales Manager' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_sales
  u.location = milwaukee
  u.roles << employee
  u.manager = greta
  u.save
end
iqbal = User.new({ email: 'iqbal@foo.com', title: 'Sales Rep' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_sales
  u.location = milwaukee
  u.roles << employee
  u.manager = han
  u.save
end
jose = User.new({ email: 'jose@foo.com', title: 'Accountant' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_finance
  u.location = milwaukee
  u.roles << accountant
  u.manager = bhavik
  u.save
end
lola = User.new({ email: 'lola@foo.com', title: 'CTO' }).tap do |u|
  u.organization = foo_industries
  u.teams << foo_leadership
  u.location = crestone
  u.roles << admin
  u.manager = alice
  u.save
end

10.times do
  Project.create(
    {
      team: foo_industries.teams.sample,
      location: [nyc, milwaukee, crestone].sample,
      name: Faker::IndustrySegments.sub_sector
    }
  )
end

100.times do
  project = Project.all.sample
  Expense.create(
    {
      employee: foo_industries.employees.where({ location: project.location }).sample,
      amount: Faker::Number.within(range: 500..100_000),
      project: project,
      description: Faker::Hipster.sentence(word_count: 2, random_words_to_add: 0)
    }
  )
end

bar_inc = Organization.create({ name: 'Bar, Inc.' })

bar_leadership = Team.create({ organization: bar_inc, name: 'Leadership' })

matz = User.new({ email: 'matz@foo.com', title: 'CEO' }).tap do |u|
  u.organization = bar_inc
  u.teams << bar_leadership
  u.location = nyc
  u.roles << admin
  u.save
end

project = Project.create(
  {
    team: bar_leadership,
    location: nyc,
    name: 'Hire first employee'
  }
)

Expense.create(
  {
    employee: matz,
    amount: Faker::Number.within(range: 500..100_000),
    project: project,
    description: 'Hire a copywriter to create some job postings.'
  }
)
