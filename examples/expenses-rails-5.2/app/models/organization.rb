# frozen_string_literal: true

class Organization < ApplicationRecord
  has_many :employees, class_name: 'User'
  has_many :teams
  has_many :projects, through: :teams
end
