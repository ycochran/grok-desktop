cask "grok-desktop" do
  version "0.1.0"
  sha256 "9088dbc2a09cba259c0edb6e63d67daccb6fb41f79194579be3227da113a9da8"

  url "https://github.com/OWNER/REPO/releases/download/v0.1.0/Grok-Desktop-for-macOS-v0.1.0.dmg"
  name "Grok Desktop for macOS"
  desc "Single-window macOS shell for Grok with local notes, prompt snippets, and workspaces"
  homepage "https://github.com/OWNER/REPO"

  depends_on macos: ">= :ventura"

  app "Grok Desktop for macOS.app"
end
