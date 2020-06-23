# frozen_string_literal: true

class Team < ApplicationRecord
  belongs_to :organization
  has_and_belongs_to_many :members, class_name: 'User'
  has_many :projects
end
