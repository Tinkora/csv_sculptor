# frozen_string_literal: true

require "open3"
require "optparse"
require "pathname"

REQUIRED = %w[
  README.md README.zh-CN.md CHANGELOG.md CODE_OF_CONDUCT.md
  CODE_OF_CONDUCT.zh-CN.md LICENSE CONTRIBUTING.md CONTRIBUTING.zh-CN.md
  SECURITY.md SECURITY.zh-CN.md SUPPORT.md SUPPORT.zh-CN.md
  docs/product_spec.md docs/product_spec.zh-CN.md
].freeze

BILINGUAL_PAIRS = [
  ["README.md", "README.zh-CN.md"],
  ["CONTRIBUTING.md", "CONTRIBUTING.zh-CN.md"],
  ["SECURITY.md", "SECURITY.zh-CN.md"],
  ["SUPPORT.md", "SUPPORT.zh-CN.md"],
  ["CODE_OF_CONDUCT.md", "CODE_OF_CONDUCT.zh-CN.md"],
  ["docs/product_spec.md", "docs/product_spec.zh-CN.md"]
].freeze

TEXT_EXTENSIONS = %w[.css .html .js .json .lock .md .rb .toml .yaml .yml].freeze
UNSUPPORTED_CLAIM = /(?:production[- ]ready|production ready).{0,40}(?:MCP server)|(?:MCP server).{0,40}(?:production[- ]ready|production ready)|(?:built[- ]in|included?|ships? with|provides?).{0,20}(?:MCP server|PDF export)/i
UTF8_BOM = "\xEF\xBB\xBF".b.freeze

options = { root: Dir.pwd }
OptionParser.new { |parser| parser.on("--root PATH") { |path| options[:root] = path } }.parse!

root = Pathname.new(File.expand_path(options[:root]))
errors = []
output, git_error, status = Open3.capture3("git", "-C", root.to_s, "ls-files", "-z")
abort("git ls-files failed: #{git_error.strip}") unless status.success?
tracked = output.split("\0").reject(&:empty?)

REQUIRED.each do |relative|
  errors << "missing required file: #{relative}" unless tracked.include?(relative) && root.join(relative).file?
end

text_files = tracked.select do |relative|
  path = root.join(relative)
  path.file? && (TEXT_EXTENSIONS.include?(path.extname.downcase) || path.basename.to_s == "LICENSE")
end

text_files.sort.each do |relative|
  path = root.join(relative)
  bytes = path.binread
  errors << "UTF-8 BOM is forbidden: #{relative}" if bytes.start_with?(UTF8_BOM)
  text = bytes.force_encoding(Encoding::UTF_8)
  unless text.valid_encoding?
    errors << "invalid UTF-8: #{relative}"
    next
  end

  if path.extname.downcase == ".md"
    errors << "unsupported public capability claim in #{relative}" if text.match?(UNSUPPORTED_CLAIM)
  end
end

tracked.grep(/\.md\z/).sort.each do |relative|
  path = root.join(relative)
  next unless path.file?

  text = path.read(encoding: "UTF-8")
  text.scan(/!?\[[^\]]*\]\((<[^>]+>|[^\s)]+)(?:\s+["'][^"']*["'])?\)/).flatten.each do |raw_target|
    target = raw_target.delete_prefix("<").delete_suffix(">")
    next if target.empty? || target.start_with?("#", "https://", "http://", "mailto:")

    local = target.split("#", 2).first
    next if local.empty?
    resolved = path.dirname.join(local).cleanpath.expand_path
    unless resolved.to_s == root.to_s || resolved.to_s.start_with?("#{root}/")
      errors << "relative link escapes repository in #{relative}: #{target}"
      next
    end
    errors << "broken relative link in #{relative}: #{target}" unless resolved.exist?
  end
end

BILINGUAL_PAIRS.each do |english, chinese|
  english_path = root.join(english)
  chinese_path = root.join(chinese)
  next unless english_path.file? && chinese_path.file?

  english_head = english_path.read(encoding: "UTF-8").lines.first(12).join
  chinese_head = chinese_path.read(encoding: "UTF-8").lines.first(12).join
  errors << "#{english} must link to #{File.basename(chinese)} near the top" unless english_head.include?(File.basename(chinese))
  errors << "#{chinese} must link to #{File.basename(english)} near the top" unless chinese_head.include?(File.basename(english))
end

if errors.empty?
  puts "Documentation contracts passed (#{text_files.length} tracked text files scanned)."
  exit 0
end

warn errors.uniq.join("\n")
exit 1
