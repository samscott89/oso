# frozen_string_literal: true

class User < ApplicationRecord
  has_and_belongs_to_many :roles
  has_and_belongs_to_many :teams
  belongs_to :location
  belongs_to :organization
  has_many :subordinates, class_name: 'User', foreign_key: 'manager_id'
  belongs_to :manager, class_name: 'User', optional: true
  has_many :expenses
end
