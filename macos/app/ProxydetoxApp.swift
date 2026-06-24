import SwiftUI

struct MenuItems: View {
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Button("Open statistics") {
            openWindow(id: "preferences")
        }
    }
}

@main
struct ProxydetoxApp: App {

@Environment(\.openWindow) var openWindow
@State var currentNumber: String = "1"
    var body: some Scene {
        Window("What's New", id: "preferences") {
            Text("New in this version")
        }

        MenuBarExtra( "Proxydetox", systemImage: "network")
        {
            MenuItems()
        }
    }
}
