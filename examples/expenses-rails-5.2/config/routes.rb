# frozen_string_literal: true

Rails.application.routes.draw do
  root 'expenses#index'
  post '/sessions', to: 'sessions#create', as: :login
  delete '/sessions', to: 'sessions#destroy', as: :logout
  resources :expenses
  resources :locations
  resources :organizations
  resources :projects
  resources :teams
end
