import SwiftUI

// MARK: - TextField Style

enum OffriiTextFieldStyle {
    case bordered
    case underline
}

// MARK: - OffriiTextField

struct OffriiTextField: View {
    let label: String
    @Binding var text: String
    var placeholder: String = ""
    var errorMessage: String? = nil
    var isSecure: Bool = false
    var style: OffriiTextFieldStyle = .bordered
    var keyboardType: UIKeyboardType = .default
    var textContentType: UITextContentType? = nil
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
            VStack(spacing: 0) {
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
                .padding(.vertical, style == .bordered ? 14 : 10)
                .padding(.horizontal, style == .bordered ? OffriiTheme.spacingMD : 0)

                if style == .underline {
                    Rectangle()
                        .fill(dividerColor)
                        .frame(height: 1)
                }
            }
            .background(style == .bordered ? OffriiTheme.card : Color.clear)
            .cornerRadius(style == .bordered ? OffriiTheme.cornerRadiusMD : 0)
            .overlay(
                Group {
                    if style == .bordered {
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                            .strokeBorder(borderColor, lineWidth: 1.5)
                    }
                }
            )
            .focused($isFocused)
            .animation(OffriiTheme.defaultAnimation, value: isFocused)
            .animation(OffriiTheme.defaultAnimation, value: errorMessage)

            // Error message
            if let errorMessage, !errorMessage.isEmpty {
                Text(errorMessage)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.danger)
                    .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
    }

    // MARK: - Computed Properties

    private var borderColor: Color {
        if errorMessage != nil && !(errorMessage?.isEmpty ?? true) {
            return OffriiTheme.danger
        }
        if isFocused {
            return OffriiTheme.primary
        }
        return OffriiTheme.border
    }

    private var dividerColor: Color {
        if errorMessage != nil && !(errorMessage?.isEmpty ?? true) {
            return OffriiTheme.danger
        }
        if isFocused {
            return OffriiTheme.primary
        }
        return OffriiTheme.border
    }
}

// MARK: - Preview

#if DEBUG
struct OffriiTextField_Previews: PreviewProvider {
    struct PreviewWrapper: View {
        @State private var email = ""
        @State private var password = ""
        @State private var errorField = "Adresse invalide"

        var body: some View {
            VStack(spacing: OffriiTheme.spacingMD) {
                OffriiTextField(
                    label: "Adresse e-mail",
                    text: $email,
                    placeholder: "vous@exemple.com",
                    keyboardType: .emailAddress,
                    textContentType: .emailAddress,
                    autocapitalization: .never
                )

                OffriiTextField(
                    label: "Mot de passe",
                    text: $password,
                    placeholder: "Votre mot de passe",
                    isSecure: true,
                    textContentType: .password
                )

                OffriiTextField(
                    label: "Champ avec erreur",
                    text: $email,
                    placeholder: "Saisir ici",
                    errorMessage: errorField
                )
            }
            .padding(OffriiTheme.spacingLG)
            .background(OffriiTheme.cardSurface)
        }
    }

    static var previews: some View {
        PreviewWrapper()
            .previewLayout(.sizeThatFits)
    }
}
#endif
