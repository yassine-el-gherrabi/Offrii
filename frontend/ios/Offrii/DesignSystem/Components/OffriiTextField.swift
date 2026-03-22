import SwiftUI

// MARK: - TextField Style

enum OffriiTextFieldStyle {
    case bordered
    case filled
}

// MARK: - OffriiTextField

struct OffriiTextField: View {
    let label: String
    @Binding var text: String
    var placeholder: String = ""
    var errorMessage: String?
    var isSecure: Bool = false
    var style: OffriiTextFieldStyle = .bordered
    var keyboardType: UIKeyboardType = .default
    var textContentType: UITextContentType?
    var autocapitalization: TextInputAutocapitalization = .sentences

    @FocusState private var isFocused: Bool
    @State private var showPassword: Bool = false

    var body: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
            // Label
            if !label.isEmpty {
                Text(label)
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textSecondary)
            }

            // Input field
            HStack(spacing: 0) {
                Group {
                    if isSecure && !showPassword {
                        SecureField(placeholder, text: $text)
                            .textContentType(textContentType)
                    } else {
                        TextField(placeholder, text: $text)
                            .keyboardType(keyboardType)
                            .textContentType(textContentType)
                            .textInputAutocapitalization(isSecure ? .never : autocapitalization)
                    }
                }
                .font(OffriiTypography.body)
                .foregroundColor(OffriiTheme.text)

                if isSecure {
                    Button {
                        showPassword.toggle()
                    } label: {
                        Image(systemName: showPassword ? "eye.slash.fill" : "eye.fill")
                            .foregroundColor(OffriiTheme.textMuted)
                            .font(.system(size: 16))
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.vertical, 14)
            .padding(.horizontal, OffriiTheme.spacingBase)
            .frame(minHeight: 48)
            .background(fieldBackground)
            .cornerRadius(OffriiTheme.cornerRadiusMD)
            .overlay(
                RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                    .strokeBorder(borderColor, lineWidth: style == .bordered ? 1.5 : 1)
            )
            .overlay(focusGlow)
            .focused($isFocused)
            .onChange(of: text) {
                if keyboardType == .decimalPad {
                    let filtered = text.filter { $0.isNumber || $0 == "," || $0 == "." }
                    if filtered != text { text = filtered }
                }
            }
            .animation(OffriiAnimation.defaultSpring, value: isFocused)
            .animation(OffriiAnimation.defaultSpring, value: errorMessage)

            // Error message with slide-in
            if let errorMessage, !errorMessage.isEmpty {
                HStack(spacing: OffriiTheme.spacingXS) {
                    Image(systemName: "exclamationmark.circle.fill")
                        .font(.system(size: 12))
                    Text(errorMessage)
                        .font(OffriiTypography.caption)
                }
                .foregroundColor(OffriiTheme.danger)
                .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
    }

    // MARK: - Computed Properties

    private var fieldBackground: Color {
        switch style {
        case .bordered: return OffriiTheme.card
        case .filled:   return OffriiTheme.surface
        }
    }

    private var borderColor: Color {
        if errorMessage != nil && !(errorMessage?.isEmpty ?? true) {
            return OffriiTheme.danger
        }
        if isFocused {
            return OffriiTheme.borderFocused
        }
        return style == .bordered ? OffriiTheme.border : OffriiTheme.border.opacity(0.5)
    }

    @ViewBuilder
    private var focusGlow: some View {
        if isFocused && errorMessage == nil {
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                .strokeBorder(OffriiTheme.primary.opacity(0.1), lineWidth: 4)
        }
    }
}
