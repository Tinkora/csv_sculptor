# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"

class CheckDocsTest < Minitest::Test
  CHECKER = File.expand_path("check_docs.rb", __dir__)
  PAIRS = %w[README CONTRIBUTING SECURITY SUPPORT CODE_OF_CONDUCT].freeze

  def test_valid_repository_passes
    with_repository do |root|
      result = run_checker(root)

      assert result[:status].success?, result[:output]
      assert_includes result[:output], "Documentation contracts passed"
    end
  end

  def test_missing_translation_fails
    with_repository(remove: ["SECURITY.zh-CN.md"]) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "missing required file: SECURITY.zh-CN.md"
    end
  end

  def test_missing_language_link_fails
    with_repository(overrides: { "README.md" => "# CSV Sculptor\n" }) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "README.md must link to README.zh-CN.md near the top"
    end
  end

  def test_broken_relative_link_fails
    with_repository(overrides: {
      "README.md" => "# CSV Sculptor\n\n[简体中文](README.zh-CN.md)\n\n[Missing](docs/missing.md)\n"
    }) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "broken relative link in README.md: docs/missing.md"
    end
  end

  def test_utf8_bom_fails
    with_repository(overrides: {
      "README.md" => "\xEF\xBB\xBF# CSV Sculptor\n\n[简体中文](README.zh-CN.md)\n".b
    }) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "UTF-8 BOM is forbidden: README.md"
    end
  end

  def test_unimplemented_capability_claim_fails
    with_repository(overrides: {
      "README.md" => "# CSV Sculptor\n\n[简体中文](README.zh-CN.md)\n\nIncludes a production-ready MCP server.\n"
    }) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "unsupported public capability claim in README.md"
    end
  end

  private

  def with_repository(remove: [], overrides: {})
    Dir.mktmpdir("csv_sculptor_docs_") do |root|
      files = {
        "README.md" => "# CSV Sculptor\n\n[简体中文](README.zh-CN.md)\n",
        "README.zh-CN.md" => "# CSV Sculptor\n\n[English](README.md)\n",
        "docs/product_spec.md" => "# Product specification\n\n[简体中文](product_spec.zh-CN.md)\n",
        "docs/product_spec.zh-CN.md" => "# 产品规格\n\n[English](product_spec.md)\n",
        "CHANGELOG.md" => "# Changelog\n",
        "LICENSE" => "MIT License\n"
      }
      PAIRS.each do |name|
        next if name == "README"

        files["#{name}.md"] = "# #{name}\n\n[简体中文](#{name}.zh-CN.md)\n"
        files["#{name}.zh-CN.md"] = "# #{name}\n\n[English](#{name}.md)\n"
      end
      files.merge!(overrides)
      remove.each { |path| files.delete(path) }

      files.each do |path, content|
        target = File.join(root, path)
        FileUtils.mkdir_p(File.dirname(target))
        File.binwrite(target, content)
      end
      run_git(root, "init", "--quiet")
      run_git(root, "add", "--all")
      yield root
    end
  end

  def run_checker(root)
    stdout, stderr, status = Open3.capture3(RbConfig.ruby, CHECKER, "--root", root)
    { output: stdout + stderr, status: status }
  end

  def run_git(root, *arguments)
    _stdout, stderr, status = Open3.capture3("git", "-C", root, *arguments)
    raise stderr unless status.success?
  end
end
