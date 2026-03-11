import SwiftUI

// MARK: - Toast Style

enum OffriiToastStyle {
    case success
    case error
    case warning
    case info

    var icon: String {
        switch self {
        case .success: return "checkmark.circle.fill"
        case .error:   return "xmark.circle.fill"
        case .warning: return "exclamationmark.triangle.fill"
        case .info:    return "info.circle.fill"
        }
    }

    var color: Color {
        switch self {
        case .success: return OffriiTheme.success
        case .error:   return OffriiTheme.danger
        case .warning: return OffriiTheme.warning
        case .info:    return OffriiTheme.secondary
        }
    }

    var backgroundColor: Color {
        switch self {
        case .success: return OffriiTheme.successLight
        case .error:   return OffriiTheme.dangerLight
        case .warning: return OffriiTheme.warningLight
        case .info:    return OffriiTheme.secondaryLight
        }
    }
}

// MARK: - OffriiToast

struct OffriiToast: View {
    let message: String
    let style: OffriiToastStyle
    @Binding var isPresented: Bool

    var body: some View {
        if isPresented {
            HStack(spacing: OffriiTheme.spacingMD) {
                Image(systemName: style.icon)
                    .font(.system(size: 18))
                    .foregroundColor(style.color)

                Text(message)
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.text)

                Spacer()

                Button {
                    withAnimation(OffriiAnimation.snappy) {
                        isPresented = false
                    }
                } label: {
                    Image(systemName: "xmark")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(OffriiTheme.textMuted)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingMD)
            .background(style.backgroundColor)
            .cornerRadius(OffriiTheme.cornerRadiusMD)
            .shadow(color: .black.opacity(0.08), radius: 8, y: 4)
            .padding(.horizontal, OffriiTheme.spacingBase)
            .transition(.move(edge: .bottom).combined(with: .opacity))
            .onAppear {
                DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
                    withAnimation(OffriiAnimation.snappy) {
                        isPresented = false
                    }
                }
            }
        }
    }
}

// MARK: - Toast Modifier

struct ToastModifier: ViewModifier {
    @Binding var isPresented: Bool
    let message: String
    let style: OffriiToastStyle

    func body(content: Content) -> some View {
        ZStack(alignment: .bottom) {
            content

            OffriiToast(message: message, style: style, isPresented: $isPresented)
                .padding(.bottom, OffriiTheme.spacingXL)
                .animation(OffriiAnimation.bouncy, value: isPresented)
        }
    }
}

extension View {
    func offriiToast(isPresented: Binding<Bool>, message: String, style: OffriiToastStyle = .success) -> some View {
        modifier(ToastModifier(isPresented: isPresented, message: message, style: style))
    }
}
