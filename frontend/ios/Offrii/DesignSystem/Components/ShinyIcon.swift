import SwiftUI

struct ShinyIcon: View {
    let systemName: String
    let color: Color

    @State private var shimmer = false

    var body: some View {
        let icon = Image(systemName: systemName)
            .font(.system(size: 60))

        icon
            .foregroundColor(color)
            .overlay {
                Rectangle()
                    .fill(
                        .linearGradient(
                            colors: [.clear, .white.opacity(0.45), .clear],
                            startPoint: .leading,
                            endPoint: .trailing
                        )
                    )
                    .frame(width: 28)
                    .rotationEffect(.degrees(25))
                    .offset(x: shimmer ? 80 : -80)
                    .mask { icon }
            }
            .onAppear {
                withAnimation(
                    .easeInOut(duration: 2.5)
                    .repeatForever(autoreverses: false)
                ) {
                    shimmer = true
                }
            }
    }
}
