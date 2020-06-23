# frozen_string_literal: true

require 'find'
require 'oso'

Rails.application.config.after_initialize do
  Rails.configuration.oso = Oso.new.tap do |oso|
    Find.find(Rails.root.join('app/policies').realpath) do |path|
      oso.load_file(path) if File.file?(path) && ['.pol', '.polar'].include?(File.extname(path))
    end

    Rails.application.eager_load! unless Rails.application.config.eager_load

    # TODO(gj): handle namespaced classes
    ApplicationRecord.descendants.each { |cls| oso.register_class(cls) { |**args| cls.find_by(**args) } }
  end
end
