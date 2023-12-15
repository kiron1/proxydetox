import SwiftUI

struct MenuItems: View {
    @Environment(\.openWindow) private var openWindow
    
    var body: some View {        
        Button("Preferences") {
            openWindow(id: "preferences")
        }
    }
}

@main
struct ProxydetoxApp: App {

    @Environment(\.openWindow) var openWindow
    @State var currentNumber: String = "1"
    @State var port: String = ""
    @State var pacUrl: String = ""
    @State var negotiate: Bool = false
    var body: some Scene {
        Window("Preferences", id: "preferences") {
            VStack(alignment: .leading) {
                HStack() {
                    Text("Listening port:")
                    TextField("Port", text: $port)
                }.padding()
                HStack() {
                    Text("PAC file URL:")
                    TextField("PAC URL", text: $pacUrl)
                }.padding()
                Toggle("Negotiate", isOn: $negotiate)
                    .toggleStyle(.checkbox).padding()
            }.padding()
        Divider()
        }

        MenuBarExtra( "Proxydetox", systemImage: "network")
        {
            MenuItems()
        }
    }
}
