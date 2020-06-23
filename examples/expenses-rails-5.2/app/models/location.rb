# frozen_string_literal: true

class Location < ApplicationRecord
  has_many :employees, class_name: 'User'
  has_many :projects
end
